export const url: string = "GAYVKL";

export class Student {
	constructor(public name: string, public id: i32, public grades: Array<f32>) { }
	getAverageGrade(): f32 {
		if (this.grades.length === 0) return 0.0;
		let sum: f32 = 0.0;
		for (let i = 0; i < this.grades.length; i++) {
			sum += this.grades[i];
		}
		return sum / this.grades.length;
	}
}

export function add(a: i32, b: i32): i32 {
	return a + b;
}

export function shout(): string {
	return url;
}

export function multiply(x: f32, y: f32): f32 {
	return x * y;
}

export function greet(name: string): string {
	return "Hello, " + name + "!";
}

export function isPositive(num: i32): boolean {
	return num > 0;
}

export function createStudent(name: string, id: i32): Student {
	const grades = new Array<f32>();
	grades.push(85.5);
	grades.push(90.0);
	return new Student(name, id, grades);
}

export function getStudentName(student: Student): string {
	return student.name;
}
