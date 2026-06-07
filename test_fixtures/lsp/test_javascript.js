/**
 * Example JavaScript module for syntax highlighting test.
 * @module geometry
 */
// One can use this with Split Windows as well. 
// SEE!! IT works! KINDA... [#07.06.2026] 
export class Point {
  /**
   * @param {number} x
   * @param {number} y
   */
  constructor(x, y) {
    this.x = x;
    this.y = y;
  }

  /**
   * @param {Point} other
   * @returns {number}
   */
  distanceTo(other) {
    const dx = this.x - other.x;
    const dy = this.y - other.y;
    return Math.sqrt(dx * dx + dy * dy);
  }
}

export class Triangle {
  /**
   * @param {Point} a
   * @param {Point} b
   * @param {Point} c
   */
  constructor(a, b, c) {
    this.a = a;
    this.b = b;
    this.c = c;
  }

  /** @returns {number} */
  get area() {
    const { a, b, c } = this;
    return Math.abs(
      (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y)) / 2.0
    );
  }
}

function main() {
  const origin = new Point(0, 0);
  const t = new Triangle(origin, new Point(3, 0), new Point(0, 4));
  console.log(`Area: ${t.area}`);
}

main();
