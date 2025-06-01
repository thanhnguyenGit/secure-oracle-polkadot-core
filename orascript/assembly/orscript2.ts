class CryptoValue {
    usd: f32;

    constructor(usd: f32 = 0.0) {
        this.usd = usd;
    }
}

class Input {
    bitcoin: CryptoValue;
    ethereum: CryptoValue;

    constructor() {
        this.bitcoin = new CryptoValue();
        this.ethereum = new CryptoValue();
    }

    static fromJSON(json: string): Input {
        let input = new Input();

        let btcStart = json.indexOf('"bitcoin":{"usd":') + 17;
        let btcEnd = json.indexOf("}", btcStart);
        let btcUsd = parseFloat(json.substring(btcStart, btcEnd)) as f32;

        let ethStart = json.indexOf('"ethereum":{"usd":') + 18;
        let ethEnd = json.indexOf("}", ethStart);
        let ethUsd = parseFloat(json.substring(ethStart, ethEnd)) as f32;

        input.bitcoin = new CryptoValue(btcUsd);
        input.ethereum = new CryptoValue(ethUsd);
        return input;
    }
}

class Output {
    greater: string;
    custom: Array<CryptoValue>;
    primi_i: i32;
    pmimi_f : f32;
    constructor() {
        this.custom = new Array<CryptoValue>(17);
        this.greater = "";
    }
}

export function process(json_ptr : usize, len: usize): Output {
    let json = String.UTF8.decodeUnsafe(json_ptr, len, true);
    const input = Input.fromJSON(json);
    const output = new Output();

    if (input.bitcoin.usd > input.ethereum.usd) {
        output.greater = "bitcoin";
    } else {
        output.greater = "ethereum";
    }
    output.custom.push(input.bitcoin);
    output.custom.push(input.ethereum);
    output.pmimi_f = 69;
    output.primi_i = 4200;
    return output;
}
