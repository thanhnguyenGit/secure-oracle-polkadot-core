class Input {
    a: i32;
    b: i32;
}

class Output {
    sum: i32;
    product: i32;
    pass_res: Array<i32>;

    constructor() {
        this.pass_res = new Array<i32>(20);
    }
}

export function process(input: Input): Output {
    let result = new Output();
    result.sum = calculate_sum(input.a, input.b);
    result.product = calculate_product(input.a, input.b);
    result.pass_res.push(result.sum);
    return result;
}

function calculate_sum(a: i32, b: i32): i32 {
    return a + b;
}

function calculate_product(a: i32, b: i32): i32 {
    return a * b;
}
