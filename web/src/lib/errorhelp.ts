export const errorHelp: Record<string, string> = {
	AUTH_ERROR: `
**Missing Authorization Header**

This error occurs because the \`Authorization\` header is missing in your request to the \`/oauth/v1/generate\` endpoint.

This endpoint requires **HTTP Basic Authentication** using your app’s consumer key and secret.

**How to fix:**

Set the \`Authorization\` header to:

Authorization: Basic <base64(consumer_key:consumer_secret)>

Make sure the credentials are correctly base64-encoded and included in the request.
`,

	INVALID_GRANT_TYPE: `
**Invalid \`grant_type\` Parameter**

This error occures when the \`grant_type\` parameter is provided with an invalid value.

The Daraja API only supports:

grant_type=client_credentials

**How to fix:**

Ensure the query string includes \`grant_type=client_credentials\` **exactly** as required.
`,

	MISSING_GRANT_TYPE: `
**Missing \`grant_type\` Parameter**

This error occured because the \`grant_type\` parameter is missing in the query string.

The \`/oauth/v1/generate\` endpoint requires \`grant_type=client_credentials\` to specify the authentication flow.

**How to fix:**

Append the following to your request URL:

?grant_type=client_credentials
`,

	INVALID_ACCESS_TOKEN: `
**Invalid Access Token**

This error occurs when the access token provided in the \`Authorization\` header is missing, expired, malformed, or does not match any active session.

To fix this:

1. Obtain a valid token by calling [\`/oauth/v1/generate\`](#) using your **consumer key** and **consumer secret**.
2. Include the token in the \`Authorization\` header like this:

Authorization: Bearer <access_token>

3. Ensure the token is not expired. Tokens are typically valid for **1 hour**.
`,
	INVALID_CREDENTIALS: `
**Invalid Credentials**

This error occurs when the \`Password\` field in your STK Push request is invalid or incorrectly formatted.

The \`Password\` must be a Base64-encoded string generated as:

Base64Encode(BusinessShortCode + Passkey + Timestamp)

To fix this:

- Ensure you are using the correct **BusinessShortCode** and **Passkey** associated with your app.
- Format the \`Timestamp\` as \`YYYYMMDDHHMMSS\`.
- Concatenate the fields *exactly*, without spaces or separators.
- Then Base64-encode the result and use it as the \`Password\` value.

If any part is incorrect, the request will be rejected with an invalid credentials error.
`
};

export default errorHelp;
