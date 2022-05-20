import App from './native.js'
import { camelToKebab } from './utils/case-convert.js'

const INTERNAL_ACTIONS = ['constructor', 'onLoad'];
const INTERNAL_EVENTS = ['constructor', 'open', 'close', 'error'];
let HTTP_CONTROLLERS = {};

export default {
	_dropCache: () => HTTP_CONTROLLERS = undefined,

	/**
	 * Register routes for actions of provided controller
	 * @param {import('./native.js').Class<import('.').HttpController>} ControllerClass
	 */
	registerHttpController (ControllerClass) {
		const basePath = '/' + camelToKebab(ControllerClass.name);
		const instance = HTTP_CONTROLLERS[ControllerClass.name] = new ControllerClass();

		for (const action of Object.getOwnPropertyNames(ControllerClass.prototype)) {
			if (action[0] == '_' || INTERNAL_ACTIONS.includes(action)) {
				continue;
			}

			App.registerRoute(basePath + '/' + camelToKebab(action), instance, instance[action]);
		}
	},
	/**
	 * Register request handler for path
	 * @param {string} path Route path pattern
	 * @param {string | import('.').HttpHandler} handler Handler function or name in format `Controller.action`
	 */
	registerRoute (pattern, handler) {
		if (typeof handler == 'string') {
			// todo: add config.allowDynamicRoutes and use _dropCache
			if (!HTTP_CONTROLLERS) {
				throw new Error('Dynamic registration of routes is disabled. Set "allowDynamicRoutes" to "true" in config to enable.');
			}

			const [controllerName, actionName] = handler.split('.', 2);
			const controller = HTTP_CONTROLLERS[controllerName];
			if (!controller) throw new Error(`Controller "${controllerName}" is not registered`);

			const action = controller[actionName];
			if (!action) throw new Error(`Controller "${controllerName}" does not contain "${actionName}" action`);

			App.registerRoute(pattern, controller, action);
		} else {
			App.registerRoute(pattern, null, handler);
		}
	},

	/**
	 * Register controller as a socket endpoint handler
	 * @param {import('./native.js').Class<import('.').SocketController>} ControllerClass
	 */
	 registerSocketController (ControllerClass) {
		const instance = new ControllerClass();

		/** @type {import('./native').SocketEndpoint} */
		const endpoint = {
			path: '/' + camelToKebab(ControllerClass.name),
			controller: instance,
			handlers: []
		};

		for (const event of Object.getOwnPropertyNames(ControllerClass.prototype)) {
			if (event[0] == '_' || INTERNAL_EVENTS.includes(event)) {
				continue;
			}

			endpoint.handlers.push({ event: camelToKebab(event), method: instance[event] });
		}

		App.registerSocket(endpoint);
	}
}
