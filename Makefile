specs = \
keycloak/5.0.json \
keycloak/6.0.json \
keycloak/7.0.json \
keycloak/8.0.json \
keycloak/9.0.json \
keycloak/10.0.json \
keycloak/11.0.json \
keycloak/12.0.json \
keycloak/sso-6.json \
keycloak/sso-7.3.json \
keycloak/sso-7.4.json \

html = \
keycloak/5.0.html \
keycloak/6.0.html \
keycloak/7.0.html \
keycloak/8.0.html \
keycloak/9.0.html \
keycloak/10.0.html \
keycloak/11.0.html \
keycloak/12.0.html \
keycloak/sso-6.html \
keycloak/sso-7.3.html \
keycloak/sso-7.4.html \

.PHONY : all
all : keycloak/LICENSE.txt $(specs)

.PHONY : clean
clean :
	rm keycloak/*

.PHONY : cleanAll
cleanAll : clean
	git clean -fXd

.SECONDARY: $(html)

keycloak/LICENSE.txt:
	curl https://raw.githubusercontent.com/keycloak/keycloak/master/LICENSE.txt > $@

keycloak/sso-%.html:
	curl "https://access.redhat.com/webassets/avalon/d/red-hat-single-sign-on/version-$(subst sso-,,$(basename $(notdir $@)))/restapi/" > $@

keycloak/%.html:
	curl "https://www.keycloak.org/docs-api/$(basename $(notdir $@))/rest-api/index.html" > $@

keycloak/%.json: keycloak/%.html
	(cd keycloak-openapi-transformer; cargo run --release) < $(addsuffix .html,$(basename $@)) > $@
