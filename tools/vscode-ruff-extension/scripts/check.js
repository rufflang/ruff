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

if (!Array.isArray(package_json.activationEvents) || !package_json.activationEvents.includes('onLanguage:ruff')) {
	throw new Error('Extension must activate on Ruff language files (onLanguage:ruff).');
}

const lsp_properties = package_json.contributes
	&& package_json.contributes.configuration
	&& package_json.contributes.configuration.properties;

if (!lsp_properties || !Array.isArray(lsp_properties['ruff.lsp.command'] && lsp_properties['ruff.lsp.command'].default)) {
	throw new Error('Extension must define ruff.lsp.command default command array.');
}

const command_default = lsp_properties['ruff.lsp.command'].default;
if (2 !== command_default.length || 'ruff' !== command_default[0] || 'lsp' !== command_default[1]) {
	throw new Error('ruff.lsp.command default must be ["ruff", "lsp"].');
}

console.log('Extension static checks passed.');
