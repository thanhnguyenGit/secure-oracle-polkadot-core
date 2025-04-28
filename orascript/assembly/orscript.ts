class Input {
    a : i32;
    b : i32;

}

class Ouput {
    sum : i32;
    product : i32;
}

export function process(input : Input): Ouput {
    let result = new Ouput();
    result.sum = calculate_sum(input.a,input.b);
    result.product = calculate_product(input.a,input.b);
    return result;
}

function calculate_sum(a:i32,b:i32) : i32 {
    return a + b;
}
function calculate_product(a: i32,b:i32) :i32 {
    return a * b;
}
