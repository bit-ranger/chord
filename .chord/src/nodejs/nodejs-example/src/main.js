let sha1 = require('sha1');


let argv = process.argv;
let case_args = JSON.parse(argv[2]);

let hash = sha1(argv);

let result = {
    chord_report_version: "1.0",
    frames: [
        {
            id: "id-0",
            start: Date.now(),
            end: Date.now(),
            data: {
                argv: argv,
                case_args: case_args,
                hash: hash
            }
        }
    ]
}
console.log("----content-output----");
console.log(JSON.stringify(result));