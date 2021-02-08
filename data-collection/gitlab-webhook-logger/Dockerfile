FROM alpine

COPY target/x86_64-unknown-linux-musl/release/gitlab-webhook /usr/local/bin/gitlab-webhook

ENTRYPOINT ["gitlab-webhook"]
