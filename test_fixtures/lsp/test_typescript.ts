/** Example TypeScript module for syntax highlighting test. */
// This is a Tab. 
interface Point {
  readonly x: number;
  readonly y: number;
}

function distance(a: Point, b: Point): number {
  const dx = a.x - b.x;
  const dy = a.y - b.y;
  return Math.sqrt(dx * dx + dy * dy);
}

abstract class Shape {
  constructor(public readonly name: string) {}

  abstract area(): number;

  toString(): string {
    return `${this.name} area=${this.area().toFixed(2)}`;
  }
}

class Triangle extends Shape {
  constructor(
    private readonly a: Point,
    private readonly b: Point,
    private readonly c: Point
  ) {
    super("Triangle");
  }

  area(): number {
    const { a, b, c } = this;
    return Math.abs(
      (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y)) / 2.0
    );
  }
}

const origin: Point = { x: 0, y: 0 };
const tri = new Triangle(origin, { x: 3, y: 0 }, { x: 0, y: 4 });
console.log(tri.toString());
