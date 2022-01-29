import { readdirSync, readFileSync, statSync, writeFileSync } from 'fs'
import Path from 'path'
const readJSON = path => JSON.parse(readFileSync(path).toString());

const pkg = readJSON('package.json');
pkg.optionalDependencies = {};

for (const entry of readdirSync('npm')) {
	const basePath = Path.join('npm', entry);
	const stat = statSync(basePath);
	if (stat.isDirectory()) {
		const subPkg = readJSON(Path.join(basePath, 'package.json'));
		pkg.optionalDependencies[subPkg.name] = subPkg.version;
	}
}

writeFileSync('package.json', JSON.stringify(pkg, null, '\t'));
