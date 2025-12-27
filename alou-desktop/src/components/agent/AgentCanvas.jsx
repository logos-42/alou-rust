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
    },
    ref,
  ) => {
    const rootRef = useRef(null)
    const avatarRef = useRef(null)

    useImperativeHandle(
      ref,
      () => ({
        getElement: () => rootRef.current,
        root: rootRef.current,
      }),
      [],
    )

    const handleAvatarPointerDown = (event) => {
      console.log('[AgentCanvas] handleAvatarPointerDown called', event)
      
      // 只处理左键
      if (event.button !== 0 && event.button !== undefined) {
        console.log('[AgentCanvas] Wrong button, rejecting', event.button)
        return
      }
      event.stopPropagation()
      
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
      onPointerMove?.(event)
    }

    const handleAvatarPointerUp = (event) => {
      event.stopPropagation()
      
      console.log('[AgentCanvas] Avatar pointer up', { pointerId: event.pointerId })
      
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
              isAvatarChild: avatarRef.current.contains(elementAtPoint)
            })
          }
        }
        setTimeout(checkOverlay, 1000)
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
          style={agentStyle}
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
              zIndex: 10000,
              position: 'relative',
              backgroundColor: 'rgba(255, 0, 0, 0.1)' // 临时添加，用于可视化调试
            }}
            onPointerDown={(e) => {
              console.log('[AgentCanvas] Avatar onPointerDown (React handler)', e)
              handleAvatarPointerDown(e)
            }}
            onPointerMove={handleAvatarPointerMove}
            onPointerUp={handleAvatarPointerUp}
            onMouseDown={(e) => {
              console.log('[AgentCanvas] Avatar onMouseDown (React handler)', e)
              e.stopPropagation()
            }}
            onClick={(e) => {
              console.log('[AgentCanvas] Avatar onClick (React handler)', e)
              e.stopPropagation()
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
