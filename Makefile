transformer = keycloak-openapi-transformer/target/release/keycloak-openapi-transformer
specs = keycloak/5.0.json keycloak/6.0.json keycloak/7.0.json keycloak/8.0.json keycloak/9.0.json keycloak/sso-6.json keycloak/sso-7.3.json
html = keycloak/5.0.html keycloak/6.0.html keycloak/7.0.html keycloak/8.0.html keycloak/9.0.html

.PHONY : all
all : keycloak/LICENSE.txt $(specs) example_app/dist

.PHONY : clean
clean :
	rm keycloak/*
	cd keycloak-openapi-transformer; cargo clean
	rm -r example_app/src/keycloak-client/
	rm -r example_app/dist/
	rm -r example_app/node_modules/

.SECONDARY: $(html)

keycloak/LICENSE.txt:
	curl https://raw.githubusercontent.com/keycloak/keycloak/master/LICENSE.txt > $@

keycloak/%.html:
	curl "https://www.keycloak.org/docs-api/$(basename $(notdir $@))/rest-api/index.html" > $@

keycloak/sso-%.html:
	curl "https://access.redhat.com/webassets/avalon/d/red-hat-single-sign-on/version-$(subst sso-,,$(basename $(notdir $@)))/restapi/" > $@

keycloak/%.json: keycloak/%.html $(transformer)
	$(transformer) < $(addsuffix .html,$(basename $@)) > $@

$(transformer): keycloak-openapi-transformer/src keycloak-openapi-transformer/Cargo.toml keycloak-openapi-transformer/Cargo.lock
	cd keycloak-openapi-transformer; cargo build --release

example_app/dist: keycloak/9.0.json example_app/package.json example_app/package-lock.json example_app/src/index.ts
	cd example_app; npm install
	cd example_app; npm run generate-client
	cd example_app; npm run compile