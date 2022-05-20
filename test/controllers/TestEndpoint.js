import { HttpController } from '../../index.js'

export class TestEndpoint extends HttpController {
	ping () {
		return 'pong';
	}

	get () {
		return this.query;
	}

	post () {
		return this.data;
	}

	hash () {
		return this.params.hash;
	}

	_private () { return 'secured content'; }
	exposedPrivate () { return this.controller._private(); }

    testDrop () {
        this.drop();
    }

    testRedirect () {
        this.redirect('/test-redirect');
    }
}
