import tape from 'tape'
import { testJSON } from '../utils/index.js'

tape('Query string parsing', async t => {
	await testJSON(t, 'Simple', {
		path: '/test-endpoint/get?a=b&t=true&f=false&null=&single_bool',
		code: 200,
		message: { a: 'b', t: true, f: false, null: null, single_bool: true }
	});

	await testJSON(t, 'Array', {
		path: '/test-endpoint/get?array[]=1&array[]=2&array[]=3',
		code: 200,
		message: { array: ['1', '2', '3'] }
	});

	await testJSON(t, 'Object', {
		path: '/test-endpoint/get?obj[a]=1&obj[b]=2&obj[c]=3',
		code: 200,
		message: { obj: { a: '1', b: '2', c: '3' } }
	});
});
