import { BaseController, HttpController } from '.'

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
 * Register controller and routes for its actions
 * @param name Controller name
 * @param controllerClass Controller class
 */
export function registerController<T extends BaseController<T>> (name: string, controllerClass: Class<T>): void;

/**
 * Register request handler for path
 * @param path Route path pattern
 * @param handler Handler name in format `Controller.action`
 */
export function registerRoute (path: string, handler: string): void;
/**
 * Register request handler for path
 * @param path Route path pattern
 * @param handler Handler function
 */
export function registerRoute (path: string, handler: (this: HttpController) => void): void;
