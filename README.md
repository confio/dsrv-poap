# DSRV POAP App

This is a simple "proof of attendence protocol" demo.

* Anyone can register an event
* The event has an image, name, start date and end date, and some other metadata (to be defined)
* The "owner" of the event and mint many copies of this badge, one per address
  * They cannot mint before the start
  * They cannot mint after the end
* Each attendee receives a "badge", which is indexed by (event, address)
  * The badge stores data if they were late or at the whole event
* The attendee CANNOT transfer the "badge"
* Given an event
  * See all attendees
  * Show logo and description of event
* Given a user
  * Show all badges they have
  * Show logo and name for each badge
