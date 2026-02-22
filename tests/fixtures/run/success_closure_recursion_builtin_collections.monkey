let mk = fn(a) {
  fn(b) {
    let arr = [a, b, a + b];
    {"sum": arr[2], "len": len(arr)}
  }
};
let f = mk(2);
let h = f(5);
h["sum"] + h["len"];
