import { HttpController, SocketController } from './index.js'
import App from './native.js'
import Router from './router.js'
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

Router.registerHttpController(class TestEndpoint extends HttpController {
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

Router.registerRoute('/test-custom/h{hash}.json', 'TestEndpoint.hash');
Router.registerRoute('/shutdown', () => process.exit(0));

Router.registerSocketController(class TestSocket extends SocketController {
    hello (args) {
        console.log(this, args);
    }
});
