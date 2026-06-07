import React, { useState, useEffect } from "react";

interface Point {
  x: number;
  y: number;
}

interface ShapeProps {
  vertices: Point[];
  color?: string;
}

const distance = (a: Point, b: Point): number => {
  const dx = a.x - b.x;
  const dy = a.y - b.y;
  return Math.sqrt(dx * dx + dy * dy);
};

export const ShapeViewer: React.FC<ShapeProps> = ({
  vertices,
  color = "#ff0000",
}) => {
  const [area, setArea] = useState<number>(0);

  useEffect(() => {
    if (vertices.length < 3) {
      setArea(0);
      return;
    }
    let sum = 0;
    const n = vertices.length;
    for (let i = 0; i < n; i++) {
      const j = (i + 1) % n;
      sum += vertices[i].x * vertices[j].y;
      sum -= vertices[j].x * vertices[i].y;
    }
    setArea(Math.abs(sum) / 2);
  }, [vertices]);

  return (
    <div style={{ border: `2px solid ${color}`, padding: "1rem" }}>
      <h3>Shape Viewer</h3>
      <p>Vertices: {vertices.length}</p>
      <p>Area: {area.toFixed(2)}</p>
      {area > 0 && (
        <svg width="100" height="100">
          <polygon
            points={vertices.map((p) => `${p.x},${p.y}`).join(" ")}
            fill={color}
            opacity={0.5}
          />
        </svg>
      )}
      <button onClick={() => setArea(0)}>Reset</button>
    </div>
  );
};
