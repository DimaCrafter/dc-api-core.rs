import tape from 'tape'
import { testJSON } from '../utils/index.js'

tape('Controller context', async t => {
	await testJSON(t, 'this.controller', {
		path: '/test-endpoint/exposed-private',
		code: 200,
		message: 'secured content'
	});
});
