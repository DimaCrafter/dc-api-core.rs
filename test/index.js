import tapSpec from 'tap-spec'
import { spawn } from 'child_process'
import { readJSON } from './utils/index.js'

const config = readJSON('./config.json');
const server = spawn('node', ['../test.js'], { stdio: ['ignore', 'pipe', 'inherit'] });
server.stdout.on('data', chunk => {
	if (~chunk.toString().indexOf('Server started on port ' + config.port)) {
		startTests();
	}
});

process.on('exit', () => server.kill());
process.on('SIGINT', () => process.exit(-1));

function startTests () {
	const runner = spawn('node', ['node_modules/.bin/tape', './tasks/*.test.js'], { stdio: ['ignore', 'pipe', 'inherit'] });

	let spec = tapSpec();
	runner.stdout.on('data', chunk => {
		spec.write(chunk);
	});
	runner.once('exit', () => {
		spec.end();
		process.stdout.write('\n');
		process.exit(0);
	});

	spec.pipe(process.stdout);
}
