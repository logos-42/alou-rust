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
    },
    ref,
  ) => {
    const rootRef = useRef(null)

    useImperativeHandle(
      ref,
      () => ({
        getElement: () => rootRef.current,
        root: rootRef.current,
      }),
      [],
    )

    const handlePointerDown = (event) => {
      onPointerDown?.(event)
    }

    return (
      <main
        ref={rootRef}
        className="agent-canvas"
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerLeave={onPointerLeave}
      >
        <div className="network-background">
          {Array.from({ length: 8 }).map((_, index) => (
            <div className="orb" key={index} />
          ))}
        </div>

        <div className="agent-node" style={agentStyle} onPointerDown={handlePointerDown}>
          <div className="agent-glow" />
          <div className="agent-avatar">
            <img src={agentProfile?.avatar} alt="agent" />
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

