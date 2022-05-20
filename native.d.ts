import { BaseController, HttpController, HttpHandler, SocketController } from '.'

type Class<T> = { new (...args: any[]): T };
type ControllerContextPatcher<T extends BaseController<T>> = (ctx: T, controller?: object) => T;

/**
 * Start API server
 * @param address Bind address with port
 * @param onListen Operation callback
 * @param patchContext Controller's context patcher
 * @returns Awaiter for server shutdown
 */
export function startApp (address: string, onListen: () => void, patchContext: ControllerContextPatcher): Promise<void>;

/**
 * Shutdown API server
 */
export function stopApp (): void;

/**
 * Register request handler for path with controller
 * @param pattern Route path pattern
 * @param controller Controller containing handler
 * @param handler Handler function
 */
export function registerRoute (pattern: string, controller: HttpController, handler: HttpHandler): void;
/**
 * Register request handler for path
 * @param pattern Route path pattern
 * @param controller Pass `null` to register handler without controller
 * @param handler Handler function
 */
export function registerRoute (pattern: string, controller: null, handler: HttpHandler): void;


class SocketEndpoint {
    path: string;
    controller: SocketController;
    handlers: SocketEventHandler[];
}

class SocketEventHandler {
    event: string;
    method (this: SocketController): void;
}

/** Register socket endpoint */
export function registerSocket (endpoint: SocketEndpoint): void;
