# OBS Source Service `obs-service-cargo`

This is a Rust written variant for https://github.com/openSUSE/obs-service-cargo_vendor and https://github.com/obs-service-cargo_audit.

## How to use OBS Service `cargo vendor`

Typical Rust projects have a **workspace** manifest at the **root of their project directory**.

A good example would be the [zellij](https://zellij.dev) project. Users will just depend the first Cargo.toml found in that project. Therefore, they do not need to use the 
`cargotoml` parameter for the `_service` file.

```xml
<services>
  <service name="download_files" mode="manual" />
  <service name="cargo_vendor" mode="manual">
     <param name="srctar">zellij-0.37.2.tar.gz</param>
     <param name="compression">zst</param>
     <param name="update">true</param>
  </service>
  <service name="cargo_audit" mode="manual" />
</services>
```

However, certain projects may not have a root manifest file, thus, each directory may be a separate subproject e.g. https://github.com/ibm-s390-linux/s390-tools and may need some thinking.

If projects like these cannot have a root manifest but have different subprojects, you may need to define the relative path to the other manifest files from root.

```xml
<services>
  <service name="cargo_vendor" mode="manual">
     <param name="srcdir">s390-tools</param>
     <param name="compression">zst</param>
     <param name="cargotoml">s390-tools/rust/utils/Cargo.toml</param>
     <param name="update">true</param>
  </service>
  <service name="cargo_audit" mode="manual" />
</services>
```

***NOTE*: A few projects do not really require crate dependencies. Those that are workspaces may have members that are dependencies of each or for each other.*
