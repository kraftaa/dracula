.PHONY: dump copy create-postgres-container build container-test dump-postgres

REPO := id.dkr.ecr.eu-central-1.amazonaws.com/dracula
TAG := $(shell git rev-parse HEAD)
MUSL_REPO := id.dkr.ecr.region-1.amazonaws.com/rust-musl
MUSL_TAG := nightly-2019-05-02

CTX := "--context=k3"

DOIT_POD = $(shell kubectl $(CTX) -n default get pods -l app=doit -o name)

detected_OS := $(shell uname)  # same as "uname -s"
# ifeq ($(detected_OS),Darwin)
    BASE64_DECODE_FLAG := "-D"

PGPASSWORD = $(shell bash -c 'kubectl $(CTX) -n production get secret app-postgres-direct-secret -o json | jq -r .data.APP_POSTGRES_PASSWORD | base64 $(BASE64_DECODE_FLAG)')
PGHOST     = $(shell bash -c 'kubectl $(CTX) -n production get secret app-postgres-direct-secret -o json | jq -r .data.APP_POSTGRES_HOST     | base64 $(BASE64_DECODE_FLAG)')
PGUSER     = $(shell bash -c 'kubectl $(CTX) -n production get secret app-postgres-direct-secret -o json | jq -r .data.APP_POSTGRES_USER     | base64 $(BASE64_DECODE_FLAG)')
BACKUP_FILE := production-2019-09-09-full.dump.tar
#BACKUP_FILE := dracula-dump.tar

TABLES := -t addresses \
   -t currencies \
   -t invoices \
   -t milestones \
   -t organizations \
   -t proposals \
   -t providers \
   -t purchase_orders \
   -t shippings \
   -t taxes \
   -t turn_around_times \
   -t users \
   -t orders \
   -t requests \
   -t notes \
   -t timepoints \
   -t wares \
   -t ratings \


DUMP_CMD := set -x; \
     rm -f $(BACKUP_FILE); \
 	PGPASSWORD=$(PGPASSWORD) pg_dump -h $(PGHOST).production --username=$(PGUSER) $(PGUSER) --create --clean -f $(BACKUP_FILE) -F t $(TABLES); \
 	rm -f $(BACKUP_FILE).gz; \
 	gzip $(BACKUP_FILE); \
 	du -hs $(BACKUP_FILE).gz

# This is for when Xavier/Maria need enough data to generate reports
dracula-dump-postgres:
	kubectl $(CTX)  -n default exec -it $(subst pod/,,$(DOIT_POD)) bash -- -c '$(DUMP_CMD)'

dracula-dump-to-s3: dracula-dump-postgres
	kubectl $(CTX)  -n default exec -it $(subst pod/,,$(DOIT_POD)) bash -- -c 'aws s3 cp $(BACKUP_FILE).gz s3://bucket-database-backups/$(BACKUP_FILE).gz'
	kubectl $(CTX)  -n default exec -it $(subst pod/,,$(DOIT_POD)) bash -- -c 'rm $(BACKUP_FILE).gz'

dracula-dump.tar.gz:
	aws s3 cp s3://bucket-database-backups/$(BACKUP_FILE) $(BACKUP_FILE)

reload-postgres: dracula-dump.tar.gz
	# gunzip $(BACKUP_FILE).gz
	- psql -h localhost --username=postgres postgres -c "DROP DATABASE $(PGUSER)"
	psql -h localhost --username=postgres postgres -c "CREATE DATABASE $(PGUSER)"
	psql -h localhost --username=postgres $(PGUSER) -c 'CREATE EXTENSION IF NOT EXISTS "uuid-ossp"'
	pg_restore --host=localhost --username=postgres --dbname=$(PGUSER) --no-owner $(BACKUP_FILE)
	rm $(BACKUP_FILE)

docker-compose-up:
	docker-compose up

doit-login:
	kubectl $(CTX) --context=k3 -n default exec -it $(subst pod/,,$(DOIT_POD)) bash -- -c 'aws ecr get-login --no-include-email --region=region-1 | sh'
	kubectl $(CTX) --context=k3 -n default exec -it $(subst pod/,,$(DOIT_POD)) bash -- -c 'aws ecr get-login --no-include-email --region=region-2 | sh'

cross-build:
	docker run \
	    --name rust-cross-build \
	    --rm -it \
	    -v $(PWD):/code \
	    -v $(PWD)/target/cross:/code/target \
	    -v $(PWD)/target/cross/registry-cache:/home/rust/.cargo/registry \
	    -v $(PWD)/target/cross/git-cache:/home/rust/.cargo/git \
	    $(MUSL_REPO):$(MUSL_TAG) \
	    bash -c 'cd /code && cargo build --bin=dracula --release'

kube-image: cross-build
	docker build -t $(REPO):$(TAG) .

reset-gitlab-cache:
	aws s3 rm s3://domain-gitlab-cache/gitlab_runner/dracula.tar.gz

# TODO: currently fails! Sad!
kube-image-test:
	docker run --rm -it --name=dracula-test $(REPO):$(TAG) /usr/bin/dracula -- --help

push: kube-image login
	docker push $(REPO):$(TAG)
	docker tag $(REPO):$(TAG) $(REPO):"tag"
	docker push $(REPO):"tag"
	kubectl -n production set image cronjob -l app=dracula dracula=$(REPO):$(TAG)
	curl https://sentry.domain.com/api/hooks/release/builtin/9/xx/ \
        -X POST \
        -H 'Content-Type: application/json' \
        -d '{"version": "$(TAG)"}'; \
	curl https://hooks.slack.com/services/id/id2/id3 \
	    -d '{"channel": "#rust", "text": "dracula $(TAG) is taking flight", "username": "Dracula", "icon_emoji": ":dracula2:"}'

deploy:
	- kubectl -n production create -f kubernetes/dracula-cronjob.yml
	kubectl -n production apply -f kubernetes/dracula-cronjob.yml

runone:
	make oneoff
	make get_oneoff

oneoff:
	git rev-parse --abbrev-ref HEAD
	git rev-parse HEAD
	python3 kubernetes/oneoff.py
	kubectl -n production delete job oneoff;
	kubectl -n production apply -f kubernetes/dracula-oneoff-job.yml
	curl https://hooks.slack.com/services/id/id2/id3 \
	-d '{"channel": "#rust", "text": "dracula-ONEOFF on $(TAG) build is created", "username": "Dracula", "icon_emoji": ":dracula2:"}'

get_oneoff:
	if $(shell until kubectl -n production get jobs oneoff -o jsonpath='{.status.conditions[?(@.type=="Complete")].status}' | grep True ; do sleep 1 ; done) == True; \
	then ( curl https://hooks.slack.com/services/id/id2/id3 -d "{\"channel\":\"#rust\", \"text\":\"Dracula-oneoff pod \
           	$(shell kubectl -n production describe pod oneoff | grep ^Status  | head -1 | awk '{print $2 }') \", \"username\":\"Dracula\", \"icon_emoji\":\":einstein-superhero:\"}" ); \
	fi; \
	echo "FINALLY!!"; \
	# make -n

login:
	aws ecr get-login --no-include-email --region=eu-central-1 | sh
	aws ecr get-login --no-include-email --region=us-west-2 | sh

clean:
	cargo clean
	rm -rf $(PWD)/target/cross

fmt:
	cargo +nightly fmt --all

EXCL_TABLE_DATA := --exclude-table-data=sessions \
        --exclude-table-data=events \
        --exclude-table-data=delayed_jobs \
        --exclude-table-data=script_results
FULL_DUMP_CMD := set -x; \
    rm -f $(BACKUP_FILE); \
	PGPASSWORD=$(PGPASSWORD) pg_dump --verbose -h $(PGHOST) --username=$(PGUSER) $(PGUSER) --create --no-privileges --clean -f $(BACKUP_FILE) -F t $(EXCL_TABLE_DATA); \
	rm -f $(BACKUP_FILE).gz; \
	gzip $(BACKUP_FILE); \
	du -hs $(BACKUP_FILE).gz

# This is for full app backups, suitable for recreating an app database elsewhere (used for bizarro)
full-dump-postgres:
	kubectl $(CTX) --context=k3 -n default exec -it $(subst pod/,,$(DOIT_POD)) bash -- -c '$(FULL_DUMP_CMD)'

# make create-model table=some-name  without 's' at the end
create-model:
	echo "[print_schema]\nfile = 'dracula_schemas/src/tables/$(table)s_tl.rs'" > diesel.toml
	echo "\nmod $(table)s_tl;\npub use self::$(table)s_tl::*;" >>	dracula_schemas/src/tables/mod.rs
	echo "\nmod $(table);\npub use self::$(table)::*;" >> dracula_schemas/src/models/mod.rs
	echo "\nmod $(table)s;\npub use self::$(table)s::*;" >> dracula_tasks/src/tasks/mod.rs
	diesel print-schema --database-url=postgres://postgres@localhost/app_production --only-tables -- $(table)s >  dracula_schemas/src/tables/$(table)s_tl.rs
	diesel_ext > dracula_schemas/src/models/$(table).rs
	sed s/magic/${table}/g  dracula_tasks/src/tasks/template_task > dracula_tasks/src/tasks/$(table)s.rs
