
import os
import uuid

str = """<?xml version="1.0" encoding="utf-8"?>


<?if $(var.Platform) = x64 ?>
<?define Win64 = "yes" ?>
<?define PlatformProgramFilesFolder = "ProgramFiles64Folder" ?>
<?else ?>
<?define Win64 = "no" ?>
<?define PlatformProgramFilesFolder = "ProgramFilesFolder" ?>
<?endif ?>


<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
    <Fragment>
        <DirectoryRef Id="LIB_DIR">
"""

for lib in os.listdir("./libraries"):
    str += f"\t<Directory Id=\"{lib}\" Name=\"{lib}\" />\n"

str += """
        </DirectoryRef>
    </Fragment>
    <Fragment>
        <ComponentGroup Id="libraries">
"""
i = 0

for lib in os.listdir("./libraries"):
    for file in os.listdir(f"./libraries/{lib}"):
        str += fr"""
            <Component Id="C{i}" Directory="{lib}" Win64='$(var.Win64)' Guid="{{{uuid.uuid4()}}}">
                <File Id="F{i}" KeyPath="yes" Source="libraries\{lib}\{file}" />
            </Component>
        """
        i += 1

str += """
        </ComponentGroup>
    </Fragment>
</Wix>
"""
print(str)
new_file = open("spwn/wix/libraries.wxs", mode="w")
new_file.write(str)
new_file.close()
