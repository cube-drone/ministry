c:
	docker compose down
clean:
	docker compose down

s:
	docker compose up -d
	docker container restart ministry && docker logs --follow ministry
start:
	docker compose up -d
	docker container restart ministry && docker logs --follow ministry

j:
	docker logs --follow ministry
join:
	docker logs --follow ministry