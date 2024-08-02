FROM --platform=linux/arm64 debian:bookworm-slim
RUN set -ex \
	&& apt-get update \
	&& apt-get install -qq --no-install-recommends curl jq

COPY init.sh /init.sh

ENTRYPOINT ["/init.sh"]
########################################################################
