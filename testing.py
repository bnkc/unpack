import importlib.metadata

# Replace 'requests' with the name of any installed package
package_name = "requests"

# Get the version of the package
version = importlib.metadata.version(package_name)
print(f"Version: {version}")

# Accessing other metadata
metadata = importlib.metadata.metadata(package_name)

# Print selected metadata
print(f"Metadata for {package_name}:")
for key in ["Name", "Version", "Author", "Author-email", "Summary", "License"]:
    print(f"{key}: {metadata.get(key)}")

# List all entry points provided by the package
print("\nEntry Points:")
entry_points = importlib.metadata.entry_points(
    group=None, name=None, package=package_name
)
for entry in entry_points:
    print(f"  {entry}")

# List all files installed by the package
print("\nFiles:")
files = importlib.metadata.files(package_name)
for file in files:
    print(f"  {file}")
