type UnBase<T, B> = {
	[K in keyof T as K extends keyof B ? never : K]: T[K]
};

export class BaseController<SubType> {
	/** Information about client IP-address */
	address: {
		version: 4 | 6,
		value: string
	}

	/** Parsed query string */
	query?: { [key: string]: any };
	/** Get request header */
	header (name: string): string;
	/** Set response header value */
	header (name: string, value: string): void;
	/** Unset response header */
	header (name: string, value: null): void;

	/** Contains all fields and methods of current controller */
	controller: UnBase<this, SubType>;
}

export class HttpController extends BaseController<HttpController> {
	/** Parsed request payload */
	data?: any;
	/** Parameters parsed from route path */
	params?: { [param: string]: string };

	/**
	 * Send response to client and close connection
	 * @param data Payload to send
	 * @param code HTTP response code, by default `200`
	 * @param isPure If `true`, then data will send without transformations, otherwise data will be serialized, by default `false`
	 */
	send (data: any, code?: number, isPure?: boolean): void;

	/** Drop the connection without responding to the request */
	drop (): void;
	/** Redirect user to specified URL */
	redirect (url: string): void;
}

export class SocketController extends BaseController<SocketController> {
	// todo
}

export type HttpHandler = (this: HttpController) => any;
