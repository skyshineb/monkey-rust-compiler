let newAdder = fn(a) {
  fn(b) { a + b }
};
let addTwo = newAdder(2);
addTwo(5);
