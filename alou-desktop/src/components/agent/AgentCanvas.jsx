import React, { forwardRef, useImperativeHandle, useRef } from 'react'
import './AgentCanvas.css'

const AgentCanvas = forwardRef(
  (
    {
      agentProfile,
      agentStyle,
      onPointerDown,
      onPointerMove,
      onPointerUp,
      onPointerLeave,
      onAgentActivate,
      onOpenSkills,
    },
    ref,
  ) => {
    const rootRef = useRef(null)
    const avatarRef = useRef(null)
    const isDraggingRef = useRef(false)
    const dragStartTimeRef = useRef(0)
    const dragStartPosRef = useRef({ x: 0, y: 0 })

    useImperativeHandle(
      ref,
      () => ({
        getElement: () => rootRef.current,
        root: rootRef.current,
      }),
      [],
    )

    const handleAvatarPointerDown = (event) => {
      // 只在开发环境记录详细日志
      if (process.env.NODE_ENV === 'development') {
        console.log('[AgentCanvas] handleAvatarPointerDown called', {
          button: event.button,
          type: event.type,
          time: Date.now()
        })
      }
      
      // 只处理左键拖动（右键在 onContextMenu 中处理）
      // 与 useAgentDrag 保持一致：如果 button 是 undefined（某些指针事件），也允许通过
      if (event.button !== undefined && event.button !== 0) {
        if (process.env.NODE_ENV === 'development') {
          console.log('[AgentCanvas] Wrong button, rejecting', event.button)
        }
        return
      }
      event.stopPropagation()
      
      // 记录拖动开始状态
      isDraggingRef.current = false
      dragStartTimeRef.current = Date.now()
      dragStartPosRef.current = { x: event.clientX, y: event.clientY }
      
      // 在头像容器元素上设置指针捕获
      if (avatarRef.current && typeof avatarRef.current.setPointerCapture === 'function') {
        try {
          avatarRef.current.setPointerCapture(event.pointerId)
        } catch (e) {
          // 忽略错误
        }
      }
      
      onPointerDown?.(event)
    }

    const handleAvatarPointerMove = (event) => {
      // 如果设置了指针捕获，事件会在头像元素上触发
      // 只在开发环境记录详细日志
      if (process.env.NODE_ENV === 'development' && dragStartTimeRef.current > 0) {
        console.log('[AgentCanvas] Avatar pointer move', {
          clientX: event.clientX,
          clientY: event.clientY,
          time: Date.now()
        })
      }
      
      onPointerMove?.(event)
    }

    const handleAvatarPointerUp = (event) => {
      event.stopPropagation()
      
      // 只在开发环境记录详细日志
      if (process.env.NODE_ENV === 'development') {
        console.log('[AgentCanvas] Avatar pointer up', {
          pointerId: event.pointerId,
          button: event.button,
          time: Date.now()
        })
      }
      
      // 释放指针捕获
      if (avatarRef.current && typeof avatarRef.current.releasePointerCapture === 'function') {
        try {
          avatarRef.current.releasePointerCapture(event.pointerId)
        } catch (e) {
          // 忽略错误
        }
      }
      
      onPointerUp?.(event)
    }

    const handleNodeClick = (event) => {
      // 如果点击的不是头像，触发原来的激活逻辑
      if (!event.target.closest('.agent-avatar')) {
        event.stopPropagation()
        onAgentActivate?.()
      }
    }

    // 添加全局测试函数（仅开发环境）
    React.useEffect(() => {
      if (process.env.NODE_ENV === 'development') {
        window.testAvatarClick = () => {
          console.log('[AgentCanvas] Manual test: Simulating right click on avatar')
          if (avatarRef.current) {
            const event = new MouseEvent('contextmenu', {
              bubbles: true,
              cancelable: true,
              button: 2,
              buttons: 2
            })
            avatarRef.current.dispatchEvent(event)
          }
        }
        
        window.testAvatarDrag = () => {
          console.log('[AgentCanvas] Manual test: Simulating drag on avatar')
          if (avatarRef.current) {
            const downEvent = new PointerEvent('pointerdown', {
              bubbles: true,
              cancelable: true,
              button: 0,
              buttons: 1,
              clientX: 100,
              clientY: 100
            })
            avatarRef.current.dispatchEvent(downEvent)
            
            setTimeout(() => {
              const moveEvent = new PointerEvent('pointermove', {
                bubbles: true,
                cancelable: true,
                button: 0,
                buttons: 1,
                clientX: 150,
                clientY: 150
              })
              avatarRef.current.dispatchEvent(moveEvent)
            }, 100)
          }
        }
        
        return () => {
          delete window.testAvatarClick
          delete window.testAvatarDrag
        }
      }
    }, [])
    
    // 在组件挂载时检查元素
    React.useEffect(() => {
      console.log('[AgentCanvas] Component mounted, checking avatarRef', avatarRef.current)
      if (avatarRef.current) {
        const rect = avatarRef.current.getBoundingClientRect()
        const computedStyle = window.getComputedStyle(avatarRef.current)
        console.log('[AgentCanvas] Avatar element info:', {
          hasRef: !!avatarRef.current,
          className: avatarRef.current.className,
          pointerEvents: computedStyle.pointerEvents,
          zIndex: computedStyle.zIndex,
          position: computedStyle.position,
          isolation: computedStyle.isolation,
          rect: { top: rect.top, left: rect.left, width: rect.width, height: rect.height }
        })
        
        // 添加一个测试点击事件
        const testClick = (e) => {
          console.log('[AgentCanvas] Test click handler triggered!', e)
        }
        avatarRef.current.addEventListener('click', testClick, true)
        avatarRef.current.addEventListener('pointerdown', (e) => {
          console.log('[AgentCanvas] Direct pointerdown listener triggered!', e)
        }, true)
        
        // 检查是否有其他元素覆盖
        const checkOverlay = () => {
          if (avatarRef.current) {
            const rect = avatarRef.current.getBoundingClientRect()
            const centerX = rect.left + rect.width / 2
            const centerY = rect.top + rect.height / 2
            const elementAtPoint = document.elementFromPoint(centerX, centerY)
            console.log('[AgentCanvas] Element at avatar center:', elementAtPoint, {
              className: elementAtPoint?.className,
              tagName: elementAtPoint?.tagName,
              isAvatar: elementAtPoint === avatarRef.current,
              isAvatarChild: avatarRef.current.contains(elementAtPoint),
              avatarZIndex: computedStyle.zIndex,
              elementZIndex: elementAtPoint ? window.getComputedStyle(elementAtPoint).zIndex : 'N/A'
            })
          
            // 如果仍然被覆盖，尝试强制提升z-index
            if (elementAtPoint && elementAtPoint !== avatarRef.current && !avatarRef.current.contains(elementAtPoint)) {
              console.log('[AgentCanvas] Avatar is still covered, trying to increase z-index')
              avatarRef.current.style.zIndex = '100000'
            }
          }
        }
        
        // 多次检查，因为布局可能随时间变化
        setTimeout(checkOverlay, 500)
        setTimeout(checkOverlay, 2000)
        setTimeout(checkOverlay, 5000)
        
        return () => {
          if (avatarRef.current) {
            avatarRef.current.removeEventListener('click', testClick, true)
          }
        }
      } else {
        console.warn('[AgentCanvas] avatarRef.current is null!')
      }
    }, [])

    return (
      <main
        ref={rootRef}
        className="agent-canvas"
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerLeave={onPointerLeave}
        onPointerDown={(e) => {
          console.log('[AgentCanvas] Main canvas pointerdown', e.target, e.target.className)
        }}
      >
        <div className="network-background">
          {Array.from({ length: 8 }).map((_, index) => (
            <div className="orb" key={index} />
          ))}
        </div>

        <div
          className="agent-node"
          style={{
            ...agentStyle,
            zIndex: 99, // 略低于头像
            position: 'relative',
            outline: 'none',
            border: 'none',
            background: 'none'
          }}
          onClick={handleNodeClick}
          onPointerDown={(e) => {
            // agent-node pointerdown
          }}
        >
          <div className="agent-glow" />
          <div 
            ref={avatarRef}
            className="agent-avatar"
            style={{ 
              pointerEvents: 'auto', 
              zIndex: 100, // 合理的z-index，避免覆盖其他重要元素
              position: 'relative',
              isolation: 'isolate', // 创建新的层叠上下文
              outline: 'none',
              border: 'none',
              background: 'none'
            }}
            onPointerDown={(e) => {
              // 只在开发环境记录详细日志
              if (process.env.NODE_ENV === 'development') {
                console.log('[AgentCanvas] Avatar onPointerDown (React handler)', {
                  button: e.button,
                  type: e.type,
                  target: e.target.className,
                  time: Date.now()
                })
              }
              e.stopPropagation()
              handleAvatarPointerDown(e)
            }}
            onPointerMove={(e) => {
              e.stopPropagation()
              handleAvatarPointerMove(e)
            }}
            onPointerUp={(e) => {
              e.stopPropagation()
              handleAvatarPointerUp(e)
            }}
            onMouseDown={(e) => {
              console.log('[AgentCanvas] Avatar onMouseDown (React handler)', e)
              e.stopPropagation()
            }}
            onClick={(e) => {
              console.log('[AgentCanvas] Avatar onClick (React handler)', e)
              e.stopPropagation()
            }}
            onContextMenu={(e) => {
              // 只在开发环境记录详细日志
              if (process.env.NODE_ENV === 'development') {
                console.log('[AgentCanvas] Avatar onContextMenu (React handler)', {
                  button: e.button,
                  type: e.type,
                  target: e.target.className,
                  time: Date.now(),
                  hasOnOpenSkills: !!onOpenSkills
                })
                console.log('[AgentCanvas] Calling onOpenSkills callback')
              }
              e.preventDefault()
              e.stopPropagation()
              onOpenSkills?.()
            }}
            onTouchStart={(e) => {
              console.log('[AgentCanvas] Avatar onTouchStart (React handler)', e)
            }}
          >
            <img src={agentProfile?.avatar} alt="agent" draggable="false" style={{ pointerEvents: 'none' }} />
          </div>
          <div className="agent-label">
            <h2>{agentProfile?.name}</h2>
          </div>
        </div>
      </main>
    )
  },
)

AgentCanvas.displayName = 'AgentCanvas'

export default AgentCanvas
