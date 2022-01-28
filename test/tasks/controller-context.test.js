import tape from 'tape'
import { testJSON, testPlain } from '../utils/index.js'

tape('Controller context', async t => {
	await testJSON(t, 'this.controller', {
		path: '/test-endpoint/exposed-private',
		code: 200,
		message: 'secured content'
	});

	await testPlain(t, 'this.drop', {
		path: '/test-endpoint/test-drop',
		code: -1,
		message: 'Error: socket hang up'
	});

	await testPlain(t, 'this.redirect', {
		path: '/test-endpoint/test-redirect',
		code: 302,
		message: '',
		resHeaders: { location: '/test-redirect' }
	});
});
