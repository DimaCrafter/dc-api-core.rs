import { HttpController, SocketController } from './index.js'
import App from './native.js'
// import config from './test/config.json'
const config = { port: 6080 };

function onListen () {
    console.log('Server started on port ' + config.port);
    // setTimeout(stopApp, 5000);
}

/** @type {import('./native').ControllerContextPatcher<HttpController & SocketController>} */
function patchContext (ctx, controller) {
    if (controller) {
        let controllerProxy;
        Object.defineProperty(ctx, 'controller', {
            get () {
                if (!controllerProxy) {
                    controllerProxy = {};
                    const { prototype } = controller.constructor;
                    for (const key of Object.getOwnPropertyNames(prototype)) {
                        if (key == 'constructor') continue;

                        const prop = prototype[key];
                        if (typeof prop == 'function') {
                            controllerProxy[key] = prop.bind(this);
                        } else {
                            controllerProxy[key] = controller[key] || prop;
                        }
                    }
                }

                return controllerProxy;
            }
        });
    }

    return ctx;
}

App.startApp('0.0.0.0:' + config.port, onListen, patchContext).then(() => {
    console.log('Dispose!');
});

App.registerController('Test', class Test extends HttpController {
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

App.registerController('TestEndpoint', class TestEndpoint extends HttpController {
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
});

App.registerRoute('/test-custom/h{hash}.json', 'TestEndpoint.hash');
App.registerRoute('/shutdown', () => process.exit(0));
