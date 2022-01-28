import { existsSync } from 'fs'
import { createRequire } from 'module';
import Path from 'path'
import { fileURLToPath } from 'url';

const bindings = {
	win32: {
		x64: 'win32-x64-msvc'
	},
	darwin: {
		x64: 'darwin-x64'
	},
	linux: {
		x64: 'linux-x64-gnu'
	}
};

// todo: Use log module
const platform = bindings[process.platform];
if (!platform) {
	console.log(`Can't load dc-api-core binary: platform ${process.platform} is unsupported`);
	process.exit(-1);
}

const triple = platform[process.arch];
if (!triple) {
	console.log(`Can't load dc-api-core binary: architecture ${process.arch} is unsupported on ${process.platform}`);
	process.exit(-1);
}

let binding;
try {
	const localPath = Path.join(Path.dirname(fileURLToPath(import.meta.url)), 'native', 'bin', 'dc-api-core.' + triple + '.node');
	if (existsSync(localPath)) {
		const require = createRequire(import.meta.url);
		binding = require(localPath);
	} else {
		binding = await import('@dpwd/dc-api-core-' + triple);
	}
} catch (err) {
	console.log('Can`t load dc-api-core binary: ' + err);
	process.exit(-1);
}

export default binding
