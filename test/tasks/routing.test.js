import tape from 'tape'
import { testPlain, testJSON } from '../utils/index.js'

tape('Routing', async t => {
	await testPlain(t, '404 error', {
		path: '/random-not-found',
		code: 404,
		message: 'API endpoint not found'
	});

	const hash = (~~(Math.random() * 16**4)).toString(16).padStart(4, '0');
	await testJSON(t, 'Custom route', {
		path: `/test-custom/h${hash}.json`,
		code: 200,
		message: hash
	});

	await testPlain(t, 'Private handlers', {
		path: '/test-endpoint/_private',
		code: 404,
		message: 'API endpoint not found'
	});
});
