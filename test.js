const { addController, registerRoute, startApp, stopApp } = require('.');

startApp('0.0.0.0:8082', () => {
    console.log('Listening');
    // setTimeout(stopApp, 5000);
}).then(() => {
    console.log('Dispose!');
});

addController('Test', class Test {
    exampleEndpoint () {
        return 'result';
    }

    action () {
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

registerRoute('/test-{subAction}', 'Test.action');
