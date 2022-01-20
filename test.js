const { addController, registerRoute, startApp, stopApp } = require('.');
const config = require('./test/config.json');

startApp('0.0.0.0:' + config.port, () => {
    console.log('Server started on port ' + config.port);
    // setTimeout(stopApp, 5000);
}).then(() => {
    console.log('Dispose!');
});

addController('Test', class Test {
    exampleEndpoint () {
        return 'result';
    }

    _action () {
        this.header('X-Test', '123');
        return {
            params: this.params,
            host: this.header('Host'),
            address: this.address,
            query: this.query
        };
    }

    errorTest () {
        throw new Error("something unexpected");
    }

    old () {
        this.send('NotFound', 404, true);
    }
});

addController('TestEndpoint', class TestEndpoint {
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
});

registerRoute('/test-custom/h{hash}.json', 'TestEndpoint.hash');
registerRoute('/shutdown', () => process.exit(0));
