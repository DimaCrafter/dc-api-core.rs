const { existsSync } = require('fs')
const Path = require('path')

function loadBinding (triple) {
	try {
		if (existsSync(Path.join(__dirname, '@dpwd/dc-api-core.' + triple + '.node'))) {
			module.exports = require('./@dpwd/dc-api-core.' + triple + '.node')
		} else {
			module.exports = require('@dpwd/dc-api-core-' + triple)
		}
	} catch (err) {
		console.log('Can`t load dc-api-core binary: ' + err); // todo: use log module
		process.exit(-1);
	}
}

switch (process.platform) {
	case 'win32':
		switch (process.arch) {
			case 'x64':
				loadBinding('win32-x64-msvc');
				break;
			default:
				throw new Error(`Unsupported architecture on Windows: ${process.arch}`)
		}
		break
	case 'darwin':
		switch (process.arch) {
			case 'x64':
				loadBinding('darwin-x64');
				break;
			default:
				throw new Error(`Unsupported architecture on macOS: ${process.arch}`);
		}
		break
	case 'linux':
		switch (process.arch) {
			case 'x64':
				loadBinding('linux-x64-gnu');
				break;
			default:
				throw new Error(`Unsupported architecture on Linux: ${process.arch}`)
		}
		break;
	default:
		throw new Error('Unsupported platform: ' + process.platform)
}
