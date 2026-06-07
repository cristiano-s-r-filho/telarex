#!/usr/bin/env python3
"""Example Python module for syntax highlighting""" 
import sys
from dataclasses import dataclass
from typing import List, Optional


@dataclass
class Point:
    x: float
    y: float

    def distance_to(self, other: "Point") -> float:
        dx = self.x - other.x
        dy = self.y - other.y
        return (dx * dx + dy * dy) ** 0.5


class Shape:
    def __init__(self, name: str, vertices: List[Point]) -> None:
        self.name = name
        self.vertices = vertices

    @property
    def area(self) -> float:
        raise NotImplementedError

    def __str__(self) -> str:
        return f"{self.name}({len(self.vertices)} vertices)"


class Triangle(Shape):
    def __init__(self, a: Point, b: Point, c: Point) -> None:
        super().__init__("Triangle", [a, b, c])

    @property
    def area(self) -> float:
        (a, b, c) = self.vertices
        return abs(
            (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y)) / 2.0
        )


def load_shapes(path: str) -> List[Shape]:
    shapes: List[Shape] = []
    if not os.path.exists(path):
        return shapes

    with open(path, "r") as f:
        for line in f:
            parts = line.strip().split(",")
            if len(parts) >= 3:
                p = Point(float(parts[0]), float(parts[1]))
                shapes.append(p)
    return shapes


if __name__ == "__main__":
    origin = Point(0.0, 0.0)
    t = Triangle(origin, Point(3.0, 0.0), Point(0.0, 4.0))
    print(f"Triangle area: {t.area}")
    sys.exit(0)
