# sudo puppet apply install_dependentapps.pp

# This is only tested on ubuntu 14.04
package { 'libtext-template-perl': ensure => present }
package { 'libxml-libxml-perl': ensure => present }

