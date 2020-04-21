import { DefaultApi } from "./keycloak-client/api";
import { exit } from "process";
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

  const api = new DefaultApi(
    { accessToken: token.access_token },
    "http://localhost:8080/auth/admin/realms"
  );

  const clients = await api.realmClientsGet("master");

  console.log("The default clients:");
  for (const client of clients.data) {
    console.log(`  ${client.clientId}`);
  }
})().catch((err) => {
  console.error(err);
  exit(1);
});
