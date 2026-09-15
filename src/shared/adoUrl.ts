const HOSTED_ROOT_ERROR =
  "Must be https://dev.azure.com/<organization> or https://<organization>.visualstudio.com";

function parseHostedUrl(value: string): URL | null {
  try {
    const url = new URL(value);
    if (
      url.protocol !== "https:" ||
      url.username ||
      url.password ||
      // URL.search/hash hide empty delimiters, but the backend rejects their presence.
      url.href.includes("?") ||
      url.href.includes("#") ||
      (url.port && url.port !== "443")
    ) {
      return null;
    }
    return url;
  } catch {
    return null;
  }
}

function pathSegments(url: URL): string[] {
  const segments = url.pathname.split("/").slice(1);
  if (segments[segments.length - 1] === "") segments.pop();
  return segments;
}

/** Validates a hosted Azure DevOps organization root used by CLI-backed feeds. */
export function validateHostedAdoOrganization(value: string): string | null {
  if (!value) return null;
  const url = parseHostedUrl(value);
  if (!url) return HOSTED_ROOT_ERROR;

  const host = url.hostname.toLowerCase();
  const segments = pathSegments(url);
  if (host === "dev.azure.com") {
    return segments.length === 1 && segments[0] ? null : HOSTED_ROOT_ERROR;
  }

  const suffix = ".visualstudio.com";
  if (!host.endsWith(suffix)) return HOSTED_ROOT_ERROR;
  const organization = host.slice(0, -suffix.length);
  return organization && !organization.includes(".") && segments.length === 0
    ? null
    : HOSTED_ROOT_ERROR;
}

/** Validates a complete hosted Azure DevOps Git repository URL. */
export function validateHostedAdoRepository(value: string): string | null {
  if (!value) return null;
  const url = parseHostedUrl(value);
  if (!url) {
    return "Must be a hosted Azure DevOps repository URL without credentials, query, or fragment";
  }

  const host = url.hostname.toLowerCase();
  const segments = pathSegments(url);
  if (host === "dev.azure.com") {
    if (segments.length !== 4) return "URL must contain organization/project/_git/repository";
    if (validateHostedAdoOrganization(`https://dev.azure.com/${segments[0]}`)) {
      return HOSTED_ROOT_ERROR;
    }
    return segments[1] !== "" && segments[2] === "_git" && segments[3] !== ""
      ? null
      : "URL must contain organization/project/_git/repository";
  }

  if (validateHostedAdoOrganization(`https://${host}`)) return HOSTED_ROOT_ERROR;
  return segments.length === 3 &&
    segments[0] !== "" &&
    segments[1] === "_git" &&
    segments[2] !== ""
    ? null
    : "URL must contain project/_git/repository";
}
