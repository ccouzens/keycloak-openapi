transformer = keycloak-openapi-transformer/target/release/keycloak-openapi-transformer

json = keycloak/5.0.json keycloak/6.0.json keycloak/7.0.json keycloak/8.0.json keycloak/9.0.json keycloak/sso-6.json

html = keycloak/5.0.html keycloak/6.0.html keycloak/7.0.html keycloak/8.0.html keycloak/9.0.html keycloak/sso-6.html keycloak/sso-7.0.0.html

.PHONY : all
all : keycloak/LICENSE.txt $(json) $(html)

.PHONY : clean
clean :
	rm keycloak/*
	cd keycloak-openapi-transformer; cargo clean

keycloak/LICENSE.txt:
	curl https://raw.githubusercontent.com/keycloak/keycloak/master/LICENSE.txt > $@

keycloak/%.html:
	curl "https://www.keycloak.org/docs-api/$(basename $(notdir $@))/rest-api/index.html" > $@

keycloak/sso-6.html:
	curl https://access.redhat.com/webassets/avalon/d/red-hat-single-sign-on/version-6/restapi/ > $@

keycloak/sso-7.0.0.html:
	curl https://access.redhat.com/webassets/avalon/d/red-hat-single-sign-on/version-7.0.0/restapi/ > $@

keycloak/%.json: keycloak/%.html $(transformer)
	$(transformer) < $(addsuffix .html,$(basename $@)) > $@

$(transformer): keycloak-openapi-transformer/src keycloak-openapi-transformer/Cargo.toml keycloak-openapi-transformer/Cargo.lock
	cd keycloak-openapi-transformer; cargo build --release