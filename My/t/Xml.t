#!/usr/bin/perl -w -I..

#****h* unittest/xml.t
# NAME
#  Xml.t
#
# SYNOPSIS
#  Unit test for Xml.pm
#
# EXAMPLE
#  
#
# SEE ALSO
# 
# See http://search.cpan.org/dist/Test-Simple/lib/Test/Tutorial.pod
#
# TODO
#
# robodoc --src Xml.t --doc doc --singlefile --html
#****



# Define the number of tests to execute.
use Test::More tests => 14;

use My::Xml;

my $nStatus = 0;
my $szXmlFile = "examples/dvd_db.xml";

#****v* xml.t/f_szXmlTest
# DESCRIPTION
#  Holds the name of the test xml file.
# SEE ALSO
#   002
# SOURCE
my $f_szXmlTest="xml_test.xml";
#****

open(XML_TEST, ">$f_szXmlTest") or die("Can't open $f_szXmlTest for write $!");
print XML_TEST "
<Database>
  <Configuration>
    <WebDirectory>/var/www/localhost/dvd</WebDirectory>
    <SqlDirectory>sql</SqlDirectory>
    <GenerateScripts>autogen</GenerateScripts>
  </Configuration>
  <General>
  <DataBaseName>dvd</DataBaseName>
  <Version>1.0</Version>
  <Description>This database is for storing my DVD's</Description>
  </General>
  <BottomLinks>
    <UrlLink Name=\"SqclConstructor\">http://sourceforge.net/projects/sqlconstructor</UrlLink>
    <UrlLink Name=\"IMDB\">http://www.imdb.com</UrlLink>
  </BottomLinks>
  <table DoList=\"y\" TableSection=\"Use\">Table01</table>
  <table DoList=\"\"  TableSection=\"Use\">Table02</table>
  <table DoList=\"y\" TableSection=\"Maintenance\">Table03</table>
  <table DoList=\"\"  TableSection=\"Maintenance\">Table04</table>
</Database>
";
close(XML_TEST);

# -------------------------------------------------- SRS-CLI-002

# ----------------------------------------------------------------------------
#****u* xml.t/cli-001
# NAME
#  SRS-XML2WEB-CLI-001
# FUNCTION
#  
#  
#
# INPUTS
#  Nothing
#
# OUTPUT
#
#  
#
# RETURN VALUE
#  
#
# NOTES
#
# 
#
# SOURCE
my $root=LoadXmlTree("NOT_EXIST");
  ok( !defined($root), "LoadXmlTree(NOT_EXIST)");

 $root=LoadXmlTree($szXmlFile);
  ok( defined($root), "LoadXmlTree(" . $szXmlFile . ")");
#****



# ----------------------------------------------------------------------------
#****u* xml.t/SRS-XML2WEB-TDIR-001
# NAME
#   SRS-XML2WEB-TDIR-001
# FUNCTION
#  
#  
#
# INPUTS
#   *  -- .
#
# OUTPUT
#
#  
#
# RETURN VALUE
#  
#
# NOTES
#
# 
#
# SOURCE
 $root=LoadXmlTree($f_szXmlTest);
 my $szAnswer=GetChildDataBySingleTagName($root,"WebDirectory");
  is( $szAnswer, "/var/www/localhost/dvd", "LoadXmlTree(" . $f_szXmlTest . ")");
#****

# ----------------------------------------------------------------------------
#****u* xml.t/utGetDataHashByTagName.001
# NAME
#   utGetDataHashByTagName.001
# FUNCTION
#  
#  
#
# INPUTS
#   *  -- .
#
# OUTPUT
#
#  
#
# RETURN VALUE
#  
#
# NOTES
#
# 
#
# SOURCE
 $root=LoadXmlTree($f_szXmlTest);
 my @arAnswer=GetDataHashByTagName($root,"UrlLink",);
# print "@arAnswer\n";
  is( $#arAnswer, 1, "GetDataHashByTagName(" . $f_szXmlTest . ",\"UrlLink\")");
  is( $arAnswer[1]{"   "},  "http://www.imdb.com", "GetDataHashByTagName() Reading data field");
  is( $arAnswer[1]{"Name"}, "IMDB", "GetDataHashByTagName() Attribute read.");
#****



# ----------------------------------------------------------------------------
#****u* xml.t/utGetDataHashByTagNameAndAttribute.001
# NAME
#   utGetDataHashByTagNameAndAttribute.001
# FUNCTION
#  
#  
#
# INPUTS
#   *  -- .
#
# OUTPUT
#
#  
#
# RETURN VALUE
#  
#
# NOTES
#
# 
#
# SOURCE
 $root=LoadXmlTree($f_szXmlTest);
 @arAnswer=GetDataHashByTagNameAndAttribute($root,"table","TableSection","Use");
# print "@arAnswer\n";
  is( $#arAnswer, 1, "GetDataHashByTagNameAndAttribute(" . $f_szXmlTest . ",\"table\",\"TableSection\",\"Use\" )");
  is( $arAnswer[0]{"   "},  "Table01", "GetDataHashByTagNameAndAttribute() Reading data field");
  is( $arAnswer[0]{"DoList"}, "y", "GetDataHashByTagNameAndAttribute() Attribute read.");
#****



# ----------------------------------------------------------------------------
#****u* xml.t/utGetDataArrayByTagName.001
# NAME
#   utGetDataArrayByTagName.001
# FUNCTION
#  
#  
#
# INPUTS
#   *  -- .
#
# OUTPUT
#
#  
#
# RETURN VALUE
#  
#
# NOTES
#
# 
#
# SOURCE
 $root=LoadXmlTree($f_szXmlTest);
 @arAnswer=GetDataArrayByTagName($root,"UrlLink",);
  is( $#arAnswer, 1, "GetDataHashByTagName(" . $f_szXmlTest . ",\"UrlLink\")");
  is( $arAnswer[0], "http://sourceforge.net/projects/sqlconstructor", "GetDataHashByTagName() Reading data field");
  is( $arAnswer[1], "http://www.imdb.com", "GetDataHashByTagName() second data field read.");
#****


# ----------------------------------------------------------------------------
#****u* xml.t/utGetDataArrayByTagAndAttribute.001
# NAME
#   utGetDataArrayByTagAndAttribute.001
# FUNCTION
#  
# OUTPUT
#
# NOTES
#
# SOURCE
 $root=LoadXmlTree($f_szXmlTest);
 @arAnswer=GetDataArrayByTagAndAttribute($root,"table","TableSection","Use");
  is( $#arAnswer, 1, "GetDataArrayByTagAndAttribute(" . $f_szXmlTest . ",\"table\",\"TableSection\",\"Use\")");
  is( $arAnswer[1], "Table02", "GetDataArrayByTagAndAttribute() Reading data field");
#****

unlink("xml_test.xml");
