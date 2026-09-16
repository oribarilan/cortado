import { describe, expect, it } from "vitest";
import { validateHostedAdoOrganization, validateHostedAdoRepository } from "./adoUrl";

describe("hosted Azure DevOps URL validation", () => {
  it("accepts hosted organization roots", () => {
    for (const value of [
      "https://dev.azure.com/acme",
      "https://dev.azure.com:443/acme/",
      "https://acme.visualstudio.com",
      "https://acme.visualstudio.com:443/",
    ]) {
      expect(validateHostedAdoOrganization(value), value).toBeNull();
    }
  });

  it("rejects untrusted and non-root organization URLs", () => {
    for (const value of [
      "https://dev.azure.com.attacker.example/acme",
      "https://attacker.example/dev.azure.com/acme",
      "https://dev.azure.com/acme/project",
      "https://dev.azure.com",
      "https://team.extra.visualstudio.com",
      "https://dev.azure.com:444/acme",
      "https://user:secret@dev.azure.com/acme",
      "https://dev.azure.com/acme?x=1",
      "https://dev.azure.com/acme#fragment",
      "https://dev.azure.com/acme?",
      "https://dev.azure.com/acme#",
    ]) {
      expect(validateHostedAdoOrganization(value), value).not.toBeNull();
    }
  });

  it("accepts hosted repository URLs including encoded names", () => {
    for (const value of [
      "https://dev.azure.com/acme/Platform/_git/API",
      "https://dev.azure.com:443/acme/My%20Project/_git/Repo%20Name/",
      "https://acme.visualstudio.com/Platform/_git/API",
    ]) {
      expect(validateHostedAdoRepository(value), value).toBeNull();
    }
  });

  it("rejects repository URL credential-routing and shape hazards", () => {
    for (const value of [
      "https://dev.azure.com.attacker.example/acme/Platform/_git/API",
      "https://attacker.example/acme/Platform/_git/API",
      "https://dev.azure.com:444/acme/Platform/_git/API",
      "https://user:secret@dev.azure.com/acme/Platform/_git/API",
      "https://dev.azure.com/acme/Platform/_git/API?version=main",
      "https://dev.azure.com/acme/Platform/_git/API#path=/src",
      "https://dev.azure.com/acme/Platform/_git/API?",
      "https://dev.azure.com/acme/Platform/_git/API#",
      "https://dev.azure.com/acme/extra/Platform/_git/API",
    ]) {
      expect(validateHostedAdoRepository(value), value).not.toBeNull();
    }
  });
});
