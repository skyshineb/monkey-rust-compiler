let rangeDesc = fn(n) {
  let i = n;
  let acc = [];
  while (i >= 1) {
    let acc = push(acc, i);
    let i = i - 1;
  };
  acc;
};

let filter = fn(arr, f) {
  let iter = fn(a, acc) {
    if (len(a) == 0) {
      acc;
    } else {
      if (f(first(a))) {
        iter(rest(a), push(acc, first(a)));
      } else {
        iter(rest(a), acc);
      }
    }
  };
  iter(arr, []);
};

let reduce = fn(arr, f, init) {
  let iter = fn(a, acc) {
    if (len(a) == 0) {
      acc;
    } else {
      iter(rest(a), f(acc, first(a)));
    }
  };
  iter(arr, init);
};

let concat = fn(a, b) {
  reduce(b, fn(acc, x) { push(acc, x); }, a);
};

let quicksort = fn(arr) {
  if (len(arr) < 2) {
    arr;
  } else {
    let pivot = first(arr);
    let tail = rest(arr);
    let left = filter(tail, fn(x) { x < pivot; });
    let right = filter(tail, fn(x) { !(x < pivot); });
    concat(push(quicksort(left), pivot), quicksort(right));
  }
};

let SIZE = 120;
let ROUNDS = 3;

let r = 0;
let checksum = 0;
let sorted = [];

while (r < ROUNDS) {
  let data = rangeDesc(SIZE);
  let sorted = quicksort(data);
  let checksum = checksum + sorted[0] + sorted[len(sorted) - 1] + len(sorted);
  let r = r + 1;
};

puts("size =", SIZE);
puts("rounds =", ROUNDS);
puts("first =", sorted[0]);
puts("last =", sorted[len(sorted) - 1]);
puts("len =", len(sorted));
puts("checksum =", checksum);