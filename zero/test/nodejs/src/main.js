let sha1 = require('sha1');


let argv =  process.argv;
let case_args = JSON.parse(argv[2]);

let hash = sha1(argv);

let result = {
  argv: argv,
  case_args: case_args,
  hash: hash
};
console.log(JSON.stringify(result));