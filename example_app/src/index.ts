import { ClientsApi, Configuration } from "./keycloak-client";
import { Issuer } from "openid-client";

(async () => {
  const keycloakIssuer = await Issuer.discover(
    "http://localhost:8080/auth/realms/master/"
  );

  const openIdConnectClient = new keycloakIssuer.Client({
    client_id: "admin-cli",
    client_secret: "unused",
  });
  const token = await openIdConnectClient.grant({
    grant_type: "password",
    username: "admin-user",
    password: "admin-password",
  });

  const config = new Configuration({
    accessToken: token.access_token,
    basePath: "http://localhost:8080/auth/admin/realms",
  });

  const clients = await new ClientsApi(config).realmClientsGet("master");

  console.log("The default clients:");
  for (const client of clients.data) {
    console.log(`  ${client.clientId}`);
  }
})().catch((err) => {
  console.error(err);
  process.exit(1);
});
