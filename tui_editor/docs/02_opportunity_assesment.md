# Opportunity assesment

## Introduction

### Purpose

### Vocabulary

- HoQ - House of Quality.
- QFD - Quality Function Deployment

### References

## QFD - Room 1 - Voice of the stakeholders

### Voice of the Customer

- Multi screen support, to make it easier to get an overview. The screens are connected, so selecting something on one screen will affect the content on the other screens.
  - Screens or windows.
- Have an edit mode, where aspects of the architecture can be entered.
- Have a view mode, where the user can use the app to get an understanding of the architecture.
- be able to split the window into multiple panes so multiple aspects of the architecture can be viewed at the same time.
- TODO how to handle having e.g. ETCD1-3 as components in a diagram?
- Type of Edit views
  - Component
  - Viewpacket
  - Diagram
    - Context diagram
    - decomposition
    - uses
    - client-server
    - deployment
    - installation
- Save in common storage format (currently XML)
- Automatic save every x minutes
  - Probably every 1 minutes as native app.
- Implement as native app
  - On linux
- Implement as web-app
- When editing diagrams make it easy to find the components that I want to add to the diagram.
- Have a view that shows the combined information of a component. Show all viewpackets where the component is the primary.
- TODO do I want a view where I can also see all the viewpackets where the component is mentioned but the component is not the primary.
- Maybe much later support the app as a microservice solution for company wide use.
- Support multiple projects, each in their own XML file.
- Support referencing components in one project in a viewpacket in another project.
  - e.g. One project is the core AWS and another project could be the telemetry, where the telemetry references e.g. accounts in the AWS project.

### Gemba

#### Adding a new component.

- Open the xml file
- Copy the template Component xml structure, from the comment at the end of the component list
- paste the structure above the comment.
- Increment the Number in the comment
- Copy the number and paster it into the new component entry id.
- Fill out the component structure.
- Move the component into the alphabetical place in the component list.

#### Adding a viewpacket

#### Adding a deployment diagram to a viewpacket

### Voice of the Employee

### Voice of the Investor

### Voice of the Suppliers

### Voice of the Sub-contractors

### Voice of the Regulator

### Voice of the Environment

### Voice of the Community

## QFD - Room 2 - Competition