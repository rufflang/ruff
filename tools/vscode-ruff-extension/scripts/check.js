const fs = require('fs');
const path = require('path');

const extension_root = path.resolve(__dirname, '..');

const required_files = [
	'package.json',
	'extension.js',
	'language-configuration.json',
	'syntaxes/ruff.tmLanguage.json',
];

for (const relative_path of required_files) {
	const absolute_path = path.join(extension_root, relative_path);
	if (!fs.existsSync(absolute_path)) {
		throw new Error('Missing required file: ' + relative_path);
	}
}

const json_files = [
	'package.json',
	'language-configuration.json',
	'syntaxes/ruff.tmLanguage.json',
];

for (const relative_path of json_files) {
	const absolute_path = path.join(extension_root, relative_path);
	const source = fs.readFileSync(absolute_path, 'utf8');
	JSON.parse(source);
}

const package_json = JSON.parse(
	fs.readFileSync(path.join(extension_root, 'package.json'), 'utf8')
);

if (!package_json.contributes || !Array.isArray(package_json.contributes.languages)) {
	throw new Error('package.json must contribute at least one language definition.');
}

if (!package_json.contributes || !Array.isArray(package_json.contributes.grammars)) {
	throw new Error('package.json must contribute at least one grammar definition.');
}

const ruff_language = package_json.contributes.languages.find(lang => 'ruff' === lang.id);
if (!ruff_language) {
	throw new Error('package.json missing language id: ruff');
}

if (!Array.isArray(ruff_language.extensions) || !ruff_language.extensions.includes('.ruff')) {
	throw new Error('Ruff language must declare .ruff extension association.');
}

console.log('Extension static checks passed.');
