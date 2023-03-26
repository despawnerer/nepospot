# NepoSpot (Serve)

Lambda that decides whether somebody is a nepo baby, or not, using data from [`nepos.csv`](../data/nepos.csv).

## Prerequsites

* [SAM CLI](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-install.html)
* [Cargo Lambda](https://github.com/awslabs/aws-lambda-rust-runtime#getting-started)

## Deploy

### First time

```bash
make build
sam deploy --guided
```

The first command will build the source of your application. The second command will package and deploy your application to AWS, with a series of prompts.

You can find your API Gateway Endpoint URL in the output values displayed after deployment.

### Subsequent times

```bash
make build
sam deploy
```

## Fetch, tail, and filter Lambda function logs

```bash
sam logs -n NepoSpotFunction --stack-name nepospot --tail
```

## Tests

```bash
cargo test
```
