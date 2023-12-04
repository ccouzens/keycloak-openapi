specs = \
keycloak/22.0.0.json \
keycloak/23.0.0.json \

yamls = \
keycloak/5.0.yml \
keycloak/6.0.yml \
keycloak/7.0.yml \
keycloak/8.0.yml \
keycloak/9.0.yml \
keycloak/10.0.yml \
keycloak/11.0.yml \
keycloak/12.0.yml \
keycloak/12.0-patched.yml \
keycloak/13.0.yml \
keycloak/13.0-patched.yml \
keycloak/14.0.yml \
keycloak/15.0.yml \
keycloak/15.1.yml \
keycloak/16.0.yml \
keycloak/16.1.yml \
keycloak/17.0.yml \
keycloak/18.0.yml \
keycloak/19.0.0.yml \
keycloak/20.0.0.yml \
keycloak/20.0.1.yml \
keycloak/20.0.2.yml \
keycloak/20.0.3.yml \
keycloak/21.0.0.yml \
keycloak/21.0.1.yml \
keycloak/21.0.2.yml \
keycloak/21.1.0.yml \
keycloak/21.1.1.yml \
keycloak/21.1.2.yml \
keycloak/22.0.0.yml \
keycloak/23.0.0.yml \
keycloak/sso-6.yml \
keycloak/sso-7.3.yml \

html = \
keycloak/22.0.0.html \
keycloak/23.0.0.html \

.PHONY : all
all : keycloak/LICENSE.txt $(specs) $(yamls)

.PHONY : clean
clean :
	rm -f $(specs) $(html) keycloak/LICENSE.txt $(yamls)

.PHONY : cleanAll
cleanAll : clean
	git clean -fXd

.SECONDARY: $(html)

keycloak/LICENSE.txt:
	curl https://raw.githubusercontent.com/keycloak/keycloak/master/LICENSE.txt > $@

keycloak/%.html:
	curl "https://www.keycloak.org/docs-api/$(basename $(notdir $@))/rest-api/index.html" > $@

keycloak/%.json: keycloak/%.html
	(cd keycloak-openapi-transformer; cargo run --release) < $(addsuffix .html,$(basename $@)) > $@

keycloak/%.yml: keycloak/%.json
	yq --output-format=yaml -P '.' $< > $@
