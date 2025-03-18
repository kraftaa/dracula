upgrade = helm upgrade  --install dracula-helm \
	../../charts/dracula \
	--kube-context=k3 \
	--namespace=production
	--set suspend="$2"
#	--set command.arg1="$(arg)"
#upgrade = helm upgrade  --install --dry-run dracula-helm-$1 \
#	../../charts/dracula \
#	--kube-context=k3 \
#	--namespace=production
#	--set command=['"bash", "-c", "echo hi"'] \
#	--set command='echo hi' \
#	--set command[0]="bash" \
#	--set command[1]="-c" \
#	--set command[2]="$2" \
#	--set schedule="$3" \
#	--set suspend="$4"

template = helm template dracula-helm  \
	../../charts/dracula \
	--kube-context=k3 \
	--namespace=production
#	--set command[0]="bash" \
#	--set command[1]="-c" \
#	--set command[2]="$2" \
#	--set schedule="$3" \
#	--set suspend="$4" \
#	--debug
#template = helm template dracula-helm-$1  \
#all:
#	command="for TASK_GROUP in all streaming_tasks action_items  notes  refs_all refs_providers  embedded_dynamic_forms; do mkdir -p target/debug && time dracula \$$POSTGRES_URL --table=\$$TASK_GROUP ; done"; \
#	schedule="0 */12 * * *"; \
#	$(call upgrade,$@,$$command,$$schedule,false)
full:
	$(call upgrade,$@)

all:
	$(call upgrade,$@)
	#$(call upgrade,$@,$$command,$$schedule,false)
	#helm upgrade --install  --dry-run  dracula-helm  ../../charts/dracula  -n production

boost:
	$(call upgrade,$@)

events:
	$(call upgrade,$@)

inventory-items:
	$(call upgrade,$@)

inventory-variations:
	$(call upgrade,$@)

#oneoff:
	#schedule="0 0 1 1 0"; #\
#	$(call upgrade,$@,$$command,$$schedule,true)
	#kubectl --context=k3 -n production delete job/dracula-helm-oneoff
	#kubectl --context=k3 -n production create job --from=cronjob/dracula-helm-oneoff dracula-helm-oneoff

#arg='beacons' make oneoff # - the problem it provides this argument to all cj ^
oneoff:
	$(call upgrade,$@,true)
	#helm upgrade --install --dry-run dracula-helm  ../../charts/dracula  -n production --set suspend=true --set command.arg1='$(arg)'
	#kubectl -n production delete job oneoff; kubectl -n production create job oneoff --from cj/dracula-helm-oneoff-oneoff

#table_name='beacons' make start-oneoff
start-oneoff:
	rm -f dracula-oneoff.yml;
	kubectl -n production create job dracula-oneoff-job --dry-run=client --from cj/dracula-helm-oneoff -oyaml > dracula-oneoff.yml
	sed -i -e 's/table_name/$(table_name)/g' dracula-oneoff.yml
	kubectl -n production delete job dracula-oneoff-job; sleep 5; kubectl -n production apply -f dracula-oneoff.yml




