export const url: string = "GAYVKL";

export function entry_point(a: i32, b: i32): i32 {
	let add_res = add(a,b);
	return add_res;
}

function add(a: i32, b: i32): i32 {
	// let x = shout();
	let x = complex_computation(a,b);
	return a + b + x;
}

function shout(): string {
	return url
}

function complex_computation(a:i32, b:i32): i32 {
	let x = a * b / 69;
	return x;
}

