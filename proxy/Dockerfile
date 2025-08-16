FROM python:3.13.5-slim-bookworm

WORKDIR /root

COPY requirements.txt requirements.txt
COPY config /app/websockify_config
COPY start.sh /app/start.sh

RUN chmod +x /app/start.sh
RUN pip install --no-cache-dir -r requirements.txt

# Standard ports
EXPOSE 443
EXPOSE 80

CMD ["/app/start.sh"]