import http from 'http'
import { readFileSync } from 'fs'

export const TEST_ERROR = {
	REQUEST: -1,
	JSON: -2
}

/**
 * Send request
 * @param {String} path
 * @param {'json' | 'urlencoded' | 'multipart' | 'plain'} type Payload type
 * @param {any} payload
 */
export function request (path, type, payload) {
	const options = {
		method: payload ? 'POST' : 'GET'
	};

	// if (payload) {
	// 	options.headers = {
	// 		'Content-Type': 'appli'
	// 	};
	// }
	let resolve;
	const req = http.request('http://localhost:6080' + path, options, res => {
		const body = [];
		res.on('data', chunk => body.push(chunk));
		res.on('end', () => {
			resolve({
				code: res.statusCode,
				message: Buffer.concat(body),
				headers: res.headers
			});
		});
	});

	if (payload) req.write(payload);
	req.on('error', err => resolve({ code: TEST_ERROR.REQUEST, message: err.toString() }));
	req.end();

	return new Promise(resolver => resolve = resolver);
}

/**
 * @typedef {object} RequestSchema
 * @property {string} path Request path
 * @property {number} code Excepted response HTTP-code
 * @property {object} [resHeaders] Excepted response headers
 * @property {any} message Excepted response object
 */

/**
 * Send request and compare plaintext response
 * @param {import('tape').Test} t Testing context
 * @param {String} label Test case message
 * @param {RequestSchema} schema Request test schema
 */
export async function testPlain (t, label, schema) {
	const response = await request(schema.path, 'plain').then(result => {
		if (result.code > 0) result.message = result.message.toString('utf-8');
		return result;
	});

	testResponse(t, label, response, schema);
}

/**
 * Send request and compare JSON response
 * @param {import('tape').Test} t Testing context
 * @param {String} label Test case message
 * @param {Object} schema Request test schema
 * @param {String} schema.path Request path
 * @param {Number} schema.code Excepted response HTTP-code
 * @param {*} schema.message Excepted response object
 */
export async function testJSON (t, label, schema) {
	const response = await request(schema.path, 'json').then(result => {
		if (result.code > 0) {
			result.message = result.message.toString();
			try {
				result.message = JSON.parse(result.message);
			} catch (err) {
				return { code: TEST_ERROR.JSON, message: result.message, error: err.toString() };
			}
		}

		return result;
	});

	testResponse(t, label, response, schema);
}

function testResponse (t, label, response, schema) {
	const fixture = {
		code: schema.code,
		message: schema.message
	};

	if (schema.resHeaders) {
		const filteredHeaders = {};
		for (const key in schema.resHeaders) {
			filteredHeaders[key] = response.headers[key];
		}

		response.headers = filteredHeaders;
		fixture.headers = schema.resHeaders;
	} else {
		delete response.headers;
	}

	t.deepEqual(response, fixture, label);
}

export const readJSON = path => JSON.parse(readFileSync(path).toString());
