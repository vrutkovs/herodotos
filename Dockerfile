FROM registry.fedoraproject.org/fedora:31 as builder

RUN dnf install -y cargo rust openssl-devel && \
     dnf clean all

COPY . .
RUN cargo build --release

FROM registry.fedoraproject.org/fedora:31
COPY --from=builder target/release/herodotos /usr/local/bin/herodotos
ENTRYPOINT [ "/usr/local/bin/herodotos" ]
