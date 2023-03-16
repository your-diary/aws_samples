# 1. About

Sample Rust projects to access AWS via SDK.

# 2. Prerequisites

- `~/.aws/credentials` (This can be created via `aws configure` command.)

- [`cargo-lambda`](https://github.com/awslabs/aws-lambda-rust-runtime)

# 3. `./todo`

## 3.1 About

## 3.2 Architecture

```
API Gateway ---> EC2 ---> RDS
                     ---> DynamoDB
                  ↓
                  S3
```

# 4. `./lambda/`

## 4.1 About

This project creates a REST API which receives a JSON of the form `{"content": <string>}` and uploads its `content` as `<timestamp>.txt` to S3.

## 4.2 Architecture

```
user -> API Gateway -> Lambda (Rust) -> S3
```

## 4.3 Usage

1. Run tests. We expect every test passes.

    ```bash
    $ cargo test
    ```

2. Cross-compile the project for Amazon Linux 2.

    ```bash
    $ cargo lambda build --release
    ```

3. Deploy the project as a lambda function. You **do not** have to create a lambda function from the console in advance; it automatically creates or updates the lambda function whose name is `lambda_test_001`. (The name can be customized via `name` property in `Cargo.toml`).

    ```bash
    $ cargo lambda deploy
    ```

4. Access [*S3 console*](https://s3.console.aws.amazon.com/s3/buckets?region=ap-northeast-1) to create a bucket called `bucket-test-001-a` with the default settings.

5. Access [*Lambda console*](https://ap-northeast-1.console.aws.amazon.com/lambda/home?region=ap-northeast-1#/functions).

    1. Select `lambda_test_001`.

    2. Select `Configuration` > `Permissions` > `Execution role` and click the name of the role (e.g. `cargo-lambda-role-...`).

    3. Add `AmazonS3FullAccess` role.

6. Access [*API Gateway console*](https://ap-northeast-1.console.aws.amazon.com/apigateway/main/apis?region=ap-northeast-1).

    1. Create a new REST API called `test_gateway_001`.

    2. Select `Actions` > `Create Method` to create a `POST` method and bind it to `lambda_test_001`.

    3. After creating the `POST` method, click `Method Request` and change the value of `API Key Required` to `true`.

    4. Select `Actions` > `Deploy API` to deploy the API. The name of a stage is arbitrary but let's use `testing` for convenience. After that, you are redirected to `Stage Editor`. Change the values of `Rate` and `Burst` as you want. You may also want to memorize the `Invoke URL` which is the URL for this API.

    5. Select `Usage Plans` in the sidebar to start creating a usage plan called `Testing`. First set `Rate`, `Burst` and `Quota` as you want, and then click `Add API Stage` to bind the `testing` stage of `test_gateway_001` to the plan.

    6. Select `API Keys` in the sidebar and select `Actions` > `Create API key` to create an API key. Then click `Add to Usage Plan` to bind it to the `Testing` plan.

7. Call the API with the key.

    ```bash
    $ curl \
        -H 'x-api-key: <API key>' \
        -d '{"content": "hello"}' \
        <URL>
    ```

    ```json
    {"status":"success","filename":"1678969418940.txt"}
    ```

## 4.4 References

- [*Using the AWS SDK for Rust in AWS Lambda function*](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/lambda.html) (a bit outdated)

- [*`lambda_runtime` crate*](https://docs.rs/lambda_runtime/latest/lambda_runtime/index.html)

<!-- vim: set spell: -->

